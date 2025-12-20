<script lang="ts">
	import { page } from '$app/stores';
	import { TitlebarButtons } from '$lib/components/ui/titlebar-buttons';
	import { default as StarlightIcon } from '$lib/assets/starlight.svg?component';
	import { ArrowLeft, ArrowRight } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';
	import { browser } from '$app/environment';

	// Get error status safely
	const errorStatus = $derived(($page.status as number) || 500);
	const errorMessage = $derived(
		$page.error?.message || 'An unexpected error occurred. Please try again later.'
	);

	const isTauri =
		browser &&
		typeof (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !==
			'undefined';
</script>

<div class="error-layout">
	<!-- Simple Titlebar -->
	<div
		data-tauri-drag-region
		class="titlebar flex h-12 items-center border-b border-border bg-card"
	>
		<div data-tauri-drag-region class="flex items-center gap-2 p-3">
			<StarlightIcon class="pointer-events-none h-5 w-5 text-foreground" />
			<div data-tauri-drag-region-exclude class="ml-3 flex items-center gap-1">
				<Button variant="navigation" aria-label="Go back" onclick={() => history.back()}>
					<ArrowLeft />
				</Button>
				<Button variant="navigation" aria-label="Go forward" onclick={() => history.forward()}>
					<ArrowRight />
				</Button>
			</div>
		</div>
		{#if isTauri}
			<section data-tauri-drag-region class="relative z-10 ml-auto flex items-center">
				<div class="mr-3 flex"></div>
				<TitlebarButtons />
			</section>
		{/if}
	</div>

	<!-- Error Content -->
	<div class="error-content flex items-center justify-center p-8">
		<div class="error-card w-full max-w-2xl rounded-lg border border-border bg-card p-8 shadow-lg">
			<div class="mb-6 text-center">
				<h1 class="mb-2 text-6xl font-bold text-destructive">
					{errorStatus}
				</h1>
				<h2 class="mb-4 text-2xl font-semibold text-foreground">
					{errorStatus === 404 ? 'Page Not Found' : 'Oops! Something went wrong'}
				</h2>
			</div>

			<div class="mb-6 text-center">
				<p class="text-base text-muted-foreground">
					{errorMessage}
				</p>
			</div>

			{#if errorStatus === 404}
				<div class="text-center">
					<Button href="/" variant="default" size="default">Go back home</Button>
				</div>
			{:else}
				<div class="flex justify-center gap-3">
					<Button variant="outline" size="default" onclick={() => window.history.back()}>
						Go back
					</Button>
					<Button href="/" variant="default" size="default">Go home</Button>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.error-layout {
		height: 100vh;
		width: 100vw;
		display: flex;
		flex-direction: column;
		background-color: var(--background);
		overflow: hidden;
	}

	.titlebar {
		flex-shrink: 0;
	}

	.error-content {
		flex: 1;
		overflow-y: auto;
	}

	:global([data-tauri-drag-region]) {
		-webkit-app-region: drag;
	}

	:global([data-tauri-drag-region-exclude]) {
		-webkit-app-region: no-drag;
	}
</style>
