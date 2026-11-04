#!/usr/bin/env node
// Sinkronkan versi dari SATU SUMBER: "version" di package.json
// -> src-tauri/tauri.conf.json dan src-tauri/Cargo.toml.
// Jalankan: npm run sync:version
import { readFileSync, writeFileSync } from 'node:fs';

const version = JSON.parse(readFileSync('package.json', 'utf8')).version;
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  throw new Error(`Versi package.json tidak valid: ${version}`);
}

const tauriConfigPath = 'src-tauri/tauri.conf.json';
const tauriConfig = readFileSync(tauriConfigPath, 'utf8');
writeFileSync(tauriConfigPath, tauriConfig.replace(
  /"version":\s*"\d+\.\d+\.\d+"/,
  `"version": "${version}"`,
));

const cargoPath = 'src-tauri/Cargo.toml';
const cargo = readFileSync(cargoPath, 'utf8');
let cargoOut = cargo.replace(/^(name = "nutrisurvey"\nversion = ")\d+\.\d+\.\d+/m, `$1${version}`);
if (cargoOut === cargo) {
  throw new Error('Paket nutrisurvey tidak ditemukan di Cargo.toml');
}
writeFileSync(cargoPath, cargoOut);

console.log(`Versi disinkronkan ke ${version} (tauri.conf.json, Cargo.toml)`);
console.log('Jangan lupa: cargo update --package nutrisurvey --precise ' + version);
