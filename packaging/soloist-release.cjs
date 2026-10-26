// Sign the exact manifest bytes fastframe-update verifies. Private key is
// supplied only by the repository's Actions secret; it is never written out.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');

function publicKey(hex) {
  assert.match(hex.trim(), /^[0-9a-f]{64}$/i);
  return crypto.createPublicKey({
    key: Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), Buffer.from(hex.trim(), 'hex')]),
    format: 'der', type: 'spki',
  });
}

function publishManifest(directory, version, keyText, trustedHex = fs.readFileSync('assets/soloist-update-public-key.hex', 'utf8')) {
  assert.match(version, /^\d+\.\d+\.\d+$/);
  const stem = `spotifast-v${version}-x86_64-pc-windows-msvc`;
  const expected = [`${stem}.zip`, `${stem}.msi`, `${stem}-setup.exe`].sort();
  const files = fs.readdirSync(directory).filter(name => fs.statSync(path.join(directory, name)).isFile()).sort();
  assert.deepEqual(files, expected, 'Release must contain only the expected Windows packages');
  const manifest = Buffer.from(files.map(name => {
    const hash = crypto.createHash('sha256').update(fs.readFileSync(path.join(directory, name))).digest('hex');
    return `${hash}  ${name}\n`;
  }).join(''), 'utf8');
  const privateKey = crypto.createPrivateKey(keyText);
  const signature = crypto.sign(null, manifest, privateKey);
  const trustedKey = publicKey(trustedHex);
  assert.equal(signature.length, 64);
  assert(crypto.verify(null, manifest, trustedKey, signature), 'Signing key does not match the committed publisher key');
  assert(!crypto.verify(null, Buffer.concat([manifest, Buffer.from('tampered')]), trustedKey, signature));
  fs.writeFileSync(path.join(directory, 'checksums.txt'), manifest);
  fs.writeFileSync(path.join(directory, 'checksums.txt.sig'), signature);
}

if (require.main === module) {
  const [directory, version] = process.argv.slice(2);
  if (!directory || !version || !process.env.SPOTIFAST_UPDATE_SIGNING_KEY) {
    throw new Error('Expected release directory, version and signing key secret');
  }
  publishManifest(directory, version, process.env.SPOTIFAST_UPDATE_SIGNING_KEY);
  console.log('Windows packages checksummed and publisher signature verified');
}

module.exports = { publicKey, publishManifest };
