import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

class GameStateService {
	#running = $state(false);
	#listeners: UnlistenFn[] = [];

	async init() {
		if (this.#listeners.length > 0) return;

		const unlisten = await listen<{ running: boolean }>('game-state-changed', (event) => {
			this.#running = event.payload.running;
		});

		this.#listeners.push(unlisten);

		const initialRunning = await invoke<boolean>('get_game_running');
		this.#running = initialRunning;
	}

	async destroy() {
		for (const unlisten of this.#listeners) {
			unlisten();
		}
		this.#listeners = [];
	}

	get running() {
		return this.#running;
	}
}

let gameStateService: GameStateService | null = null;

export function getGameState() {
	if (!gameStateService) {
		gameStateService = new GameStateService();
	}
	return gameStateService;
}

export async function initGameState() {
	const service = getGameState();
	await service.init();
	return service;
}
