const net = require('node:net');
const { randomUUID } = require('node:crypto');

const GUARDIAN_PIPE = String.raw`\\\\.\\pipe\\KidOSGuardian.v1`;
const PROTOCOL_VERSION = 1;
const MAX_MESSAGE_BYTES = 64 * 1024;

function requestGuardian(request, timeoutMs = 5000) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection(GUARDIAN_PIPE);
    const chunks = [];
    let size = 0;
    let settled = false;

    const finish = (error, value) => {
      if (settled) return;
      settled = true;
      socket.destroy();
      if (error) reject(error);
      else resolve(value);
    };

    socket.setTimeout(timeoutMs, () => finish(new Error('KidOS Guardian request timed out.')));

    socket.on('connect', () => {
      const id = randomUUID();
      const envelope = {
        version: PROTOCOL_VERSION,
        session_id: `electron-${process.pid}-${id}`,
        nonce: id,
        request,
      };
      const encoded = Buffer.from(JSON.stringify(envelope), 'utf8');
      if (encoded.length > MAX_MESSAGE_BYTES) {
        finish(new Error('KidOS Guardian request exceeded the IPC size limit.'));
        return;
      }
      socket.end(encoded);
    });

    socket.on('data', (chunk) => {
      size += chunk.length;
      if (size > MAX_MESSAGE_BYTES) {
        finish(new Error('KidOS Guardian response exceeded the IPC size limit.'));
        return;
      }
      chunks.push(chunk);
    });

    socket.on('error', (error) => finish(new Error(`KidOS Guardian is unavailable: ${error.message}`)));

    socket.on('end', () => {
      if (settled) return;
      try {
        const response = JSON.parse(Buffer.concat(chunks).toString('utf8'));
        if (!response || typeof response !== 'object') {
          throw new Error('Guardian returned an invalid response.');
        }
        if (response.type === 'error') {
          throw new Error(`${response.code || 'guardian_error'}: ${response.message || 'Guardian rejected the request.'}`);
        }
        finish(null, response);
      } catch (error) {
        finish(error instanceof Error ? error : new Error('KidOS received an invalid Guardian response.'));
      }
    });
  });
}

module.exports = { requestGuardian, GUARDIAN_PIPE, MAX_MESSAGE_BYTES };
