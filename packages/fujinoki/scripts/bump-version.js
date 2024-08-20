#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const semver = require('semver');
const pkg = require('../js/package.json');

// These values come from the invocation of release.
const increment = process.argv[2];

// Now we get the current version of the package.
const versionFilePath = path.join(__dirname, '..', '..', '..', 'version.txt');
const versionFileContents = fs.readFileSync(versionFilePath, 'utf-8');
const [currentVersion] = versionFileContents.split('\n');

// Now that we know current state, figure out what the target state is.
// If we're doing a "pre" release, set the identifier to canary
// TODO xtasks
const identifier = increment.startsWith('pre') ? 'beta' : 'latest';
const newVersion = semver.inc(currentVersion, increment, identifier);

// Parse the output semver identifier to identify which npm tag to publish to.
const parsed = semver.parse(newVersion);
const tag = parsed?.prerelease[0] || 'latest';

pkg.version = newVersion;
fs.writeFileSync(versionFilePath, `${newVersion}\n${tag}\n`);

const file = require.resolve('../js/package.json');

// TODO(kijv) use one package and both platforms target it
const knownWindowsPackages = {
  'win32 arm64 LE': 'fujinoki-windows-arm64',
  'win32 x64 LE': 'fujinoki-windows-64',
};

const knownUnixLikePackages = {
  'darwin arm64 LE': 'fujinoki-darwin-arm64',
  'darwin x64 LE': 'fujinoki-darwin-64',
  'linux arm64 LE': 'fujinoki-linux-arm64',
  'linux x64 LE': 'fujinoki-linux-64',
};

pkg.optionalDependencies = Object.fromEntries(
  Object.values({
    ...knownWindowsPackages,
    ...knownUnixLikePackages,
  })
    .sort()
    .map((x) => [x, pkg.version]),
);

fs.writeFileSync(file, `${JSON.stringify(pkg, null, 2)}\n`);
