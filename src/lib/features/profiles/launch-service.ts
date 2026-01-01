import { invoke } from '@tauri-apps/api/core';
import { join } from '@tauri-apps/api/path';
import { exists } from '@tauri-apps/plugin-fs';
import { profileService } from './profile-service';
import { settingsService } from '../settings/settings-service';
import { gameState } from './game-state-service.svelte';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { Profile } from './schema';

class LaunchService {
	async launchProfile(profile: Profile): Promise<void> {
		const settings = await settingsService.getSettings();

		if (!settings.among_us_path) throw new Error('Among Us path not configured');

		const gameExePath = await join(settings.among_us_path, 'Among Us.exe');
		const gameExists = await exists(gameExePath);
		if (!gameExists) {
			throw new Error('Among Us.exe not found at configured path');
		}

		const bepinexDll = await join(profile.path, 'BepInEx', 'core', 'BepInEx.Unity.IL2CPP.dll');
		const bepinexExists = await exists(bepinexDll);
		if (!bepinexExists) {
			throw new Error('BepInEx DLL not found. Please wait for installation to complete.');
		}

		const dotnetDir = await join(profile.path, 'dotnet');
		const coreClr = await join(dotnetDir, 'coreclr.dll');
		const coreClrExists = await exists(coreClr);
		if (!coreClrExists) {
			throw new Error('dotnet runtime not found. Please wait for installation to complete.');
		}

		await loginToEpic();

		await invoke('launch_modded', {
			gameExe: gameExePath,
			profilePath: profile.path,
			bepinexDll: bepinexDll,
			dotnetDir: dotnetDir,
			coreclrPath: coreClr
		});

		await profileService.updateLastLaunched(profile.id);
		gameState.setRunningProfile(profile.id);

		if (settings.close_on_launch) {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			getCurrentWindow().close();
		}
	}

	async launchVanilla(): Promise<void> {
		const settings = await settingsService.getSettings();

		if (!settings.among_us_path) throw new Error('Among Us path not configured');

		const gameExePath = await join(settings.among_us_path, 'Among Us.exe');
		const gameExists = await exists(gameExePath);
		if (!gameExists) {
			throw new Error('Among Us.exe not found at configured path');
		}

		await invoke('launch_vanilla', { gameExe: gameExePath });
		gameState.setRunningProfile(null);
	}
}

async function loginToEpic() {
	// Try to restore existing session first
	const restored = await invoke<boolean>('epic_try_restore_session');
	if (restored) {
		console.log('Session restored!');
		return;
	}

	// Open browser for login
	const url = await invoke<string>('get_epic_auth_url');
	console.log(`Opening URL: ${url}`);
	await openUrl(url);

	// User pastes the code from browser
	const code = prompt('Enter the authorization code:');
	if (!code) return;

	await invoke('epic_login_with_code', { code });
	console.log('Logged in!');
}

export const launchService = new LaunchService();
