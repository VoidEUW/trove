<!-- A sidebar with entries, because an entry reads whether the sidebar is collapsed from its context. -->
<script lang="ts">
	import Sidebar from '$lib/components/Sidebar.svelte';
	import SidebarEntry from '$lib/components/SidebarEntry.svelte';

	interface Props {
		collapsed?: boolean;
		width?: number;
		onselect?: () => void;
	}

	let { collapsed = $bindable(false), width = $bindable(52), onselect = () => {} }: Props = $props();
</script>

<Sidebar
	label="Main"
	title="Trove"
	bind:collapsed
	bind:width
	collapseLabel="Collapse"
	expandLabel="Expand"
	resizeLabel="Resize"
>
	<SidebarEntry href="/items" label="Items" active meta="3" />
	<SidebarEntry label="Settings" onclick={onselect} />
	<SidebarEntry label="Soon" disabled onclick={onselect} />
	<SidebarEntry href="/hidden" label="Hidden link" disabled />
</Sidebar>
