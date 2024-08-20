#!/usr/bin/env node

// @ts-check

// TODO xtasks

import path from 'path';
import { fileURLToPath } from 'url';
import { execa } from 'execa';
import { findUp, pathExists } from 'find-up';
import { copy } from 'fs-extra';
import { readFile, writeFile } from 'fs/promises';
import { humanizedArchLookup } from './npm-native-packages.mjs';

const platforms = process.argv[process.argv.length - 1];

if (platforms.endsWith('publish.mjs')) {
  console.log('missing platforms');
  process.exit(1);
}
console.log('platforms:', platforms);

(async () => {
  const rootDirectory = await findUp(
    async (directory) => {
      const hasPkgJson = await pathExists(path.join(directory, 'package.json'));
      if (hasPkgJson && path.basename(directory) === 'heian') return directory;
    },
    { type: 'directory' },
  );

  try {
    if (!rootDirectory)
      throw new Error('Could not find root directory (packages/heian)');

    const packageJson = JSON.parse(
      await readFile(path.join(rootDirectory, 'js', 'package.json'), 'utf-8'),
    );

    for (const platform of platforms.split(',')) {
      const [os, arch] = platform.split('-');

      // Copy binaries to package folders, update version, and publish
      const nativePackagesDir = path.join(rootDirectory, 'npm');

      try {
        const binaryName = `heian-${os}-${humanizedArchLookup[arch]}`.concat(
          os === 'windows' ? '.exe' : '',
        );
        await copy(
          path.join(rootDirectory, 'native', binaryName),
          path.join(
            nativePackagesDir,
            platform,
            'bin',
            'heian'.concat(os === 'windows' ? '.exe' : ''),
          ),
        );
        const nativePackageJson = JSON.parse(
          await readFile(
            path.join(nativePackagesDir, platform, 'package.json'),
            'utf-8',
          ),
        );
        nativePackageJson.version = packageJson.version;
        await writeFile(
          path.join(nativePackagesDir, platform, 'package.json'),
          JSON.stringify(nativePackageJson, null, 2),
        );
        await execa(
          'npm',
          ['pack', `${path.join(nativePackagesDir, platform)}`],
          {
            stdio: 'inherit',
          },
        );
      } catch (err) {
        // don't block publishing other versions on single platform error
        console.error('Failed to publish', platform, err);

        if (
          err.message?.includes(
            'You cannot publish over the previously published versions',
          )
        ) {
          console.error('Ignoring already published error', platform, err);
        } else {
          throw err;
        }
      }
    }
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
