#!/usr/bin/env node

// Ripped from https://github.com/vercel/turborepo/blob/main/packages/turbo/bin/turbo

/**
 * We need to run a platform-specific binary. The dependency _should_
 * have already been installed, but it's possible that it has not.
 */

const child_process = require('child_process');
const fs = require('fs');
const path = require('path');

// If we do not find the correct platform binary, should we attempt to install it?
const SHOULD_INSTALL = true;

// If we do not find the correct platform binary, should we trust calling an emulated variant?
const SHOULD_ATTEMPT_EMULATED = true;

// Relies on the fact that each tarball publishes the `package.json`.
// We can simply cd into the package directory and install there.
function installUsingNPM() {
  const packageDir = path.dirname(require.resolve('fujinoki/package'));

  // Erase "npm_config_global" so that "npm install --global" works.
  // Otherwise this nested "npm install" will also be global, and the install
  // will deadlock waiting for the global installation lock.
  const env = { ...process.env, npm_config_global: undefined };

  child_process.execSync(
    'npm install --loglevel=error --prefer-offline --no-audit --progress=false',
    { cwd: packageDir, stdio: "pipe", env }
  );
}

// npm has multiple places that we need to look for a package name.
function hasPackage(sourceObject, packageName) {
  return !!sourceObject[packageName] || !!sourceObject[`node_modules/${packageName}`];
}

// This provides logging messages as it progresses towards calculating the binary path.
function getBinaryPath() {
  // First we see if the user has configured a particular binary path.
  const FUJINOKI_BINARY_PATH = process.env.FUJINOKI_BINARY_PATH;
  if (FUJINOKI_BINARY_PATH) {
    if (!fs.existsSync(FUJINOKI_BINARY_PATH)) {
      console.error(`Fujinoki was unable to find the executable specified by FUJINOKI_BINARY_PATH:\n${FUJINOKI_BINARY_PATH}`);
      console.error();
      console.error('FUJINOKI_BINARY_PATH is intended for development use-cases. You likely want to unset the environment variable.');
      process.exit(1);
    } else {
      return FUJINOKI_BINARY_PATH;
    }
  }

  const availablePlatforms = [
    'darwin',
    'linux',
    'windows',
  ];

  const availableArchs = [
    '64',
    'arm64',
  ];

  // We need to figure out which binary to hand the user.
  // The only place where the binary can be at this point is `require.resolve`-able
  // relative to this package as it should be installed as an optional dependency.

  const { platform, arch } = process;
  const ext = platform === 'windows' ? '.exe' : '';

  // Try all places in order until we get a hit.

  // 1. The package which contains the binary we _should_ be running.
  const correctBinary = availablePlatforms.includes(platform) && availableArchs.includes(arch) ? `fujinoki-${platform}-${arch}/fujinoki${ext}` : null;
  if (correctBinary !== null) {
    try {
      return require.resolve(`${correctBinary}`);
    } catch (e) {}
  }

  // 2. Install the binary that they need just in time.
  if (SHOULD_INSTALL && correctBinary !== null) {
    console.warn('Fujinoki did not find the correct binary for your platform.');
    console.warn('We will attempt to install it now.');

    try {
      installUsingNPM();
      const resolvedPath = require.resolve(`${correctBinary}`);
      console.warn('Installation has succeeded.');
      return resolvedPath;
    } catch (e) {
      console.warn('Installation has failed.');
    }
  }

  // 3. Both Windows and macOS ARM boxes can run x64 binaries. Attempt to run under emulation.
  const alternateBinary = (arch === "arm64" && ['darwin', 'windows'].includes(platform)) ? `fujinoki-${platform}-x64/fujinoki${ext}` : null;
  if (SHOULD_ATTEMPT_EMULATED && alternateBinary !== null) {
    try {
      const resolvedPath = require.resolve(`${alternateBinary}`);
      console.warn(`Fujinoki detected that you're running:\n${platform} ${resolvedArch}.`);
      console.warn(`We were not able to find the binary at:\n${correctBinary}`);
      console.warn(`We found a possibly-compatible binary at:\n${alternateBinary}`);
      console.warn('We will attempt to run that binary.');
      return resolvedPath;
    } catch (e) {}
  }

  // We are not going to run the binary this invocation.
  // Let's give the best error message that we can.

  // Possible error scenarios:
  // - The user is on a platform/arch combination we do not support.
  // - We somehow got detection wrong and never attempted to run the _actual_ `correctBinary` or `alternateBinary`.
  // - The user doesn't have the correct packages installed for their platform.

  // Explain our detection attempt:
  console.error();
  console.error('***');
  console.error();
  console.error('Fujinoki failed to start.');
  console.error();
  console.error(`Fujinoki detected that you are running:\n${platform} ${resolvedArch}`);

  // Tell them if we support their platform at all.
  if (!availablePlatforms.includes(platform)) {
    console.error();
    console.error('Fujinoki does not presently support your platform.');
    process.exit(1);
  } else if (!availableArchs.includes(resolvedArch)) {
    if (availablePlatforms.includes(platform)) {
      console.error();
      console.error('Fujinoki supports your platform, but does not support your processor architecture.');
      process.exit(1);
    } else {
      console.error();
      console.error('Fujinoki does not either of your platform or processor architecture.');
      process.exit(1);
    }
  }

  if (correctBinary !== null) {
    console.error();
    console.error('***');
    console.error();
    console.error(`We were not able to find the binary at:\n${correctBinary}`);
    console.error();
    console.error('We looked for it at:');
    console.error(require.resolve.paths(correctBinary).join('\n'));
  }
  if (alternateBinary !== null) {
    console.error();
    console.error('***');
    console.error();
    console.error(`Your platform (${platform}) can sometimes run x86 under emulation.`);
    console.error(`We did not find a possibly-compatible binary at:\n${alternateBinary}`);
    console.error();
    console.error('We looked for it at:');
    console.error(require.resolve.paths(alternateBinary).join('\n'));
  }

  // Investigate other failure modes.

  // Has the wrong platform's binaries available.
  const availableBinaries = availablePlatforms.flatMap(platform => availableArchs.map(arch => `fujinoki-${platform}-${arch}/fujinoki${platform === 'windows' ? '.exe' : ''}`));
  const definitelyWrongBinaries = availableBinaries.filter(binary => binary !== correctBinary || binary !== correctBinary);;
  const otherInstalled = definitelyWrongBinaries.filter(binaryPath => {
    try {
      return require.resolve(binaryPath);
    } catch (e) {}
  });

  console.error();
  console.error('***');
  console.error();

  if (otherInstalled.length > 0) {
    console.error('Fujinoki checked to see if binaries for another platform are installed.');
    console.error('This typically indicates an error in sharing of pre-resolved node_modules across platforms.');
    console.error('One common reason for this is copying files to Docker.');
    console.error();
    console.error('We found these unnecessary binaries:');
    console.error(otherInstalled.join('\n'));
  } else {
    console.error('We did not find any binaries on this system.');
    console.error('This can happen if you run installation with the --no-optional flag.');
  }

  // Check to see if we have partially-populated dependencies in the npm lockfile.
  const MAX_LOOKUPS = 10;
  const availablePackages = availablePlatforms.flatMap(platform => availableArchs.map(arch => `fujinoki-${platform}-${arch}`));

  try {
    // Attempt to find project root.
    const selfPath = require.resolve('fujinoki/package');

    let previous = null;
    let current = path.join(selfPath, '..', '..', 'package-lock.json');

    for (let i = 0; previous !== current && i < MAX_LOOKUPS; i++) {
      try {
        const lockfile = fs.readFileSync(current);
        const parsedLockfile = JSON.parse(lockfile);

        const sourceObject = parsedLockfile?.dependencies ?? parsedLockfile?.packages ?? {};

        // If we don't show up in the lockfile it's the wrong lockfile.
        if (hasPackage(sourceObject, 'fujinoki')) {
          // Check to see if all of `fujinoki-<PLATFORM>-<ARCH>` is included.
          const hasAllPackages = availablePackages.every(pkg => hasPackage(sourceObject, pkg));
          if (!hasAllPackages) {
            console.error();
            console.error('***');
            console.error();
            console.error(`Fujinoki detected that your lockfile (${current}) does not enumerate all available platforms.`);
            console.error('This is likely a consequence of an npm issue: https://github.com/npm/cli/issues/4828.');

            // Let's build their repair command:
            let version = '';
            let environment = '';
            if (parsedLockfile?.packages[""]?.dependencies?.fujinoki) {
              version = `@${parsedLockfile.packages[""].dependencies.fujinoki}`;
              environment = ' --save-prod';
            } else if (parsedLockfile?.packages[""]?.devDependencies?.fujinoki) {
              version = `@${parsedLockfile.packages[""].devDependencies.fujinoki}`;
              environment = ' --save-dev';
            } else if (parsedLockfile?.packages[""]?.optionalDependencies?.fujinoki) {
              version = `@${parsedLockfile.packages[""].optionalDependencies.fujinoki}`;
              environment = ' --save-optional';
            }

            console.error();
            console.error('To resolve this issue for your repository, run:');
            console.error(`npm install fujinoki${version} --package-lock-only${environment} && npm install`);
            console.error();
            console.error('You will need to commit the updated lockfile.');
          }
          break;
        }
        break;
      } catch (e) {}

      const next = path.join(current, '..', '..', 'package-lock.json');
      previous = current;
      current = next;
    }
  } catch (e) {}

  console.error();
  console.error('***');
  console.error();
  console.error('If you believe this is an error, please include this message in your report.');

  process.exit(1);
}

// Run the binary we got.
try {
  child_process.execFileSync(
    getBinaryPath(),
    process.argv.slice(2),
    { stdio: "inherit" }
  );
} catch (e) {
  if (e?.status) process.exit(e.status);
  throw e;
}
