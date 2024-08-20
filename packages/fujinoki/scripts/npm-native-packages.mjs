#!/usr/bin/env node

// @ts-check

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

export const humanizedArchLookup = {
  amd64: '64',
  arm64: 'arm64',
};

const nodeOsLookup = {
  darwin: 'darwin',
  linux: 'linux',
  windows: 'win32',
};

const nodeArchLookup = {
  amd64: 'x64',
  arm64: 'arm64',
};

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const triples = Object.keys(nodeOsLookup).flatMap((os) => ({
  os,
  arch: Object.keys(nodeArchLookup),
}));

const version = process.argv[2];

for (const triple of triples) {
  for (const arch of triple.arch) {
    const platform = `${triple.os}-${humanizedArchLookup[arch]}`;
    const packageJson = {
      name: `fujinoki-${platform}`,
      description: `The ${platform} binary for fujinoki`,
      version,
      os: [nodeOsLookup[triple.os]],
      cpu: [nodeArchLookup[arch]],
    };

    // similar to napi-rs generate-npm-dir
    const outputPath = path.join(
      __dirname,
      '..',
      'npm',
      `${triple.os}-${nodeArchLookup[arch]}`,
    );
    fs.rmSync(outputPath, { recursive: true, force: true });
    fs.mkdirSync(path.join(outputPath, 'bin'), { recursive: true });

    // Unfortunately even though npm shims "bin" commands on Windows with auto-
    // generated forwarding scripts, it doesn't strip the ".exe" from the file name
    // first. So it's possible to publish executables via npm on all platforms
    // except Windows. I consider this a npm bug.
    //
    // My workaround is to add this script as another layer of indirection. It'll
    // be slower because node has to boot up just to shell out to the actual exe,
    // but Windows is somewhat of a second-class platform to npm so it's the best
    // I can do I think.
    if (triple.os === 'windows') {
      fs.writeFileSync(
        path.join(outputPath, 'bin', 'fujinoki'),
        `#!/usr/bin/env node

const path = require('path');
const exe = path.join(__dirname, 'fujinoki.exe');
const child_process = require('child_process');
child_process.spawnSync(exe, process.argv.slice(2), { stdio: 'inherit' });`,
      );
    }
    fs.writeFileSync(
      path.join(outputPath, 'README.md'),
      `# \`${packageJson.name}\`\n\nThis is a platform-specific binary for \`fujinoki\``,
    );
    fs.writeFileSync(
      path.join(outputPath, 'package.json'),
      JSON.stringify(packageJson, null, 2),
    );
  }
}
