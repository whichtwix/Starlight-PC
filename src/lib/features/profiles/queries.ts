import { queryOptions } from '@tanstack/svelte-query';
import { profileService } from './profile-service';
import { getGameState } from './game-state-service.svelte';

export const profileQueries = {
	all: () =>
		queryOptions({
			queryKey: ['profiles'] as const,
			queryFn: () => profileService.getProfiles()
		}),
	active: () =>
		queryOptions({
			queryKey: ['profiles', 'active'] as const,
			queryFn: () => profileService.getActiveProfile()
		}),
	hasAny: () =>
		queryOptions({
			queryKey: ['profiles', 'hasAny'] as const,
			queryFn: () => profileService.getProfiles().then((profiles) => profiles.length > 0)
		}),
	gameRunning: () =>
		queryOptions({
			queryKey: ['game', 'running'] as const,
			queryFn: async () => {
				const gameState = getGameState();
				return gameState.running;
			},
			refetchInterval: 1000
		})
};
