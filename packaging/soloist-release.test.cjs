const { test } = require('node:test');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { publishManifest, publicKey } = require('./soloist-release.cjs');

test('release manifest matches packages and fastframe raw Ed25519 signature format', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'spotifast-signing-test-'));
  try {
    const pair = crypto.generateKeyPairSync('ed25519');
    const privateText = pair.privateKey.export({ format: 'pem', type: 'pkcs8' });
    const publicHex = pair.publicKey.export({ format: 'der', type: 'spki' }).subarray(-32).toString('hex');
    const stem = 'spotifast-v0.14.3-x86_64-pc-windows-msvc';
    for (const suffix of ['.zip', '.msi', '-setup.exe']) fs.writeFileSync(path.join(directory, stem + suffix), 'dummy package ' + suffix);
    const other = crypto.generateKeyPairSync('ed25519').publicKey.export({ format: 'der', type: 'spki' }).subarray(-32).toString('hex');
    assert.throws(() => publishManifest(directory, '0.14.3', privateText, other), /does not match/);
    assert(!fs.existsSync(path.join(directory, 'checksums.txt')));
    publishManifest(directory, '0.14.3', privateText, publicHex);
    const manifest = fs.readFileSync(path.join(directory, 'checksums.txt'));
    const signature = fs.readFileSync(path.join(directory, 'checksums.txt.sig'));
    assert.equal(signature.length, 64);
    assert(crypto.verify(null, manifest, publicKey(publicHex), signature));
    assert(!crypto.verify(null, Buffer.concat([manifest, Buffer.from('changed')]), publicKey(publicHex), signature));
    for (const line of manifest.toString('utf8').trimEnd().split('\n')) {
      const [hash, name] = line.split('  ');
      assert.equal(hash, crypto.createHash('sha256').update(fs.readFileSync(path.join(directory, name))).digest('hex'));
    }
  } finally { fs.rmSync(directory, { recursive: true }); }
});

test('invalid publisher keys are rejected', () => {
  for (const key of ['', 'ab'.repeat(31), 'xx'.repeat(32)]) assert.throws(() => publicKey(key));
});
