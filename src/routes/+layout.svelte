<script lang="ts">
	import '../app.css';
	import { browser } from '$app/environment';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import AppShell from '$lib/components/layout/AppShell.svelte';
	import { initGameState } from '$lib/features/profiles/game-state-service.svelte';
	import { onDestroy, onMount } from 'svelte';

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				enabled: browser,
				staleTime: 1000 * 60 * 5, // 5 minutes - data stays fresh
				gcTime: 1000 * 60 * 10 // 10 minutes - keep in cache
			}
		}
	});
	let { children } = $props();

	let cleanup: (() => Promise<void>) | null = null;

	onMount(async () => {
		const gameState = await initGameState();
		cleanup = async () => {
			await gameState.destroy();
		};
	});

	onDestroy(async () => {
		await cleanup?.();
	});
</script>

<ModeWatcher />
<QueryClientProvider client={queryClient}>
	<AppShell>
		{@render children()}
	</AppShell>
</QueryClientProvider>
<Toaster />
