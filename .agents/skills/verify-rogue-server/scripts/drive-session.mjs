#!/usr/bin/env node
import { mkdir, writeFile } from "node:fs/promises";
import { dirname } from "node:path";

const [url, outputPath] = process.argv.slice(2);
if (!url || !outputPath) {
  console.error("Usage: drive-session.mjs <ws-url> <evidence-json>");
  process.exit(2);
}

const evidence = { url, events: [] };
const socket = new WebSocket(url);
let step = 0;
let failed = false;
const timeout = setTimeout(() => fail("Timed out waiting for WebSocket response"), 10_000);

function fail(message) {
  failed = true;
  clearTimeout(timeout);
  console.error(message);
  try { socket.close(); } catch {}
}

socket.addEventListener("open", () => evidence.events.push({ action: "connect", result: "open" }));
socket.addEventListener("message", async ({ data }) => {
  let response;
  try {
    response = JSON.parse(data);
  } catch {
    return fail("Server returned invalid JSON");
  }
  evidence.events.push({ action: evidence.events.at(-1)?.nextAction ?? "initial snapshot", response });
  if (step === 0) {
    if (response.type !== "snapshot" || response.version !== 1 || response.ended) return fail("Expected an active version-1 game snapshot");
    step = 1;
    evidence.events.at(-1).nextAction = "inspect inventory (no turn consumed)";
    socket.send(JSON.stringify({ version: 1, request_id: "verify-inventory", command: { type: "inventory" } }));
  } else if (step === 1) {
    if (response.type !== "snapshot" || response.turn !== 0 || response.ended || !response.message.startsWith("Inventory:")) return fail("Inventory must return inventory state without consuming a turn");
    step = 2;
    evidence.events.at(-1).nextAction = "wait (consume one turn)";
    socket.send(JSON.stringify({ version: 1, request_id: "verify-wait", command: { type: "wait" } }));
  } else if (step === 2) {
    if (response.type !== "snapshot" || response.turn !== 1 || response.ended) return fail("Wait must advance the active game to turn 1");
    step = 3;
    evidence.events.at(-1).nextAction = "quit (end this connection's game)";
    socket.send(JSON.stringify({ version: 1, request_id: "verify-quit", command: { type: "quit" } }));
  } else if (step === 3) {
    if (!response.ended || response.result !== "quit") return fail("Quit must return an ended response with result quit");
    step = 4;
    clearTimeout(timeout);
    try {
      await mkdir(dirname(outputPath), { recursive: true });
      await writeFile(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);
      console.log(`PASS: inventory kept turn 0; wait advanced to turn 1; quit ended game. Evidence: ${outputPath}`);
      socket.close();
    } catch (error) {
      fail(`Could not write evidence: ${error.message}`);
    }
  }
});
socket.addEventListener("error", () => fail("WebSocket connection failed"));
socket.addEventListener("close", () => {
  clearTimeout(timeout);
  if (step !== 4) process.exitCode = 1;
});