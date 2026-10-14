<script lang="ts">
	import { onMount } from 'svelte';
	import Plus from '@lucide/svelte/icons/plus';
	import {
		addItem,
		errorMessage,
		kindName,
		listItems,
		listKinds,
		onItemsChanged,
		toCommandError,
		type ItemView,
		type KindView
	} from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import DataTable, { type Column, type Sort } from '$lib/components/DataTable.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Select from '$lib/components/Select.svelte';
	import StatePage from '$lib/components/StatePage.svelte';
	import TextField from '$lib/components/TextField.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { shell } from '$lib/shell.svelte';

	let items = $state<ItemView[]>([]);
	let kinds = $state<KindView[]>([]);
	let loaded = $state(false);
	let loadError = $state<string | null>(null);
	let sort = $state<Sort | null>(null);
	let selected = $state<string | null>(null);

	// The add form.
	let open = $state(false);
	let name = $state('');
	let kindId = $state<string | null>(null);
	let quantity = $state('1');
	let nameError = $state<string | undefined>();
	let quantityError = $state<string | undefined>();
	let formError = $state<string | undefined>();
	let saving = $state(false);

	async function refetch() {
		try {
			items = await listItems();
			loadError = null;
		} catch (error) {
			loadError = errorMessage(error);
		} finally {
			loaded = true;
		}
	}

	// A write elsewhere in the app tells the views with `items_changed`; they refetch (ADR 4).
	onMount(() => {
		let stop: (() => void) | undefined;
		let gone = false;
		refetch();
		listKinds().then(
			(found) => (kinds = found),
			(error) => (loadError = errorMessage(error))
		);
		onItemsChanged(() => refetch()).then((unlisten) => (gone ? unlisten() : (stop = unlisten)));
		return () => {
			gone = true;
			stop?.();
		};
	});

	$effect(() => {
		shell.title = t('items.title');
		shell.toolbar = toolbar;
		return () => {
			shell.title = undefined;
			shell.toolbar = undefined;
		};
	});

	const kindOptions = $derived(kinds.map((kind) => ({ value: kind.id, label: kindName(kind) })));

	const columns: Column<ItemView>[] = [
		{
			id: 'name',
			header: t('items.column.name'),
			type: 'name',
			value: (item) => item.name,
			detail: (item) => item.id.slice(-4).toUpperCase(),
			sortable: true
		},
		{ id: 'kind', header: t('items.column.kind'), value: (item) => kindName(item.kind), sortable: true },
		{
			id: 'quantity',
			header: t('items.column.quantity'),
			type: 'mono',
			value: (item) => String(item.quantity),
			align: 'end',
			sortable: true
		}
	];

	const sorted = $derived.by(() => {
		if (!sort) return items;
		const direction = sort.direction === 'ascending' ? 1 : -1;
		const compare: Record<string, (a: ItemView, b: ItemView) => number> = {
			name: (a, b) => a.name.localeCompare(b.name),
			kind: (a, b) => kindName(a.kind).localeCompare(kindName(b.kind)),
			quantity: (a, b) => a.quantity - b.quantity
		};
		return [...items].sort((a, b) => (compare[sort!.column]?.(a, b) ?? 0) * direction);
	});

	function openForm() {
		name = '';
		quantity = '1';
		kindId = kinds[0]?.id ?? null;
		nameError = quantityError = formError = undefined;
		open = true;
	}

	async function save() {
		nameError = quantityError = formError = undefined;
		if (!name.trim()) nameError = t('item-form.name.required');
		const whole = /^\d+$/.test(quantity.trim()) ? Number(quantity.trim()) : NaN;
		if (!(whole >= 1)) quantityError = t('item-form.quantity.invalid');
		if (nameError || quantityError) return;

		saving = true;
		try {
			await addItem({ name, kind: kindId ?? '', quantity: whole });
			open = false;
		} catch (error) {
			// Core is the authority: whatever it refuses is shown next to the field it is about.
			const { code } = toCommandError(error);
			const message = errorMessage(error);
			if (code === 'name_required') nameError = message;
			else if (code === 'quantity_invalid') quantityError = message;
			else formError = message;
		} finally {
			saving = false;
		}
	}
</script>

{#snippet plus()}<Plus />{/snippet}
{#snippet toolbar()}
	<Button variant="primary" icon={plus} onclick={openForm}>{t('items.add')}</Button>
{/snippet}
{#snippet empty()}
	<StatePage
		kind="empty"
		title={t('items.empty.title')}
		description={t('items.empty.body')}
		primary={{ label: t('items.add'), onclick: openForm }}
	/>
{/snippet}

{#if loadError && items.length === 0}
	<StatePage
		kind="error"
		title={t('items.error.title')}
		description={loadError}
		primary={{ label: t('items.error.retry'), onclick: refetch }}
	/>
{:else if loaded}
	<DataTable
		label={t('items.title')}
		{columns}
		rows={sorted}
		rowKey={(item) => item.id}
		{sort}
		onsort={(next) => (sort = next)}
		bind:selected
		{empty}
	/>
{/if}

<Dialog bind:open title={t('item-form.title.new')}>
	<form
		id="item-form"
		class="form"
		novalidate
		onsubmit={(event) => {
			event.preventDefault();
			save();
		}}
	>
		<TextField label={t('item-form.name.label')} bind:value={name} error={nameError} />
		<Select label={t('item-form.kind.label')} options={kindOptions} bind:value={kindId} />
		<TextField
			label={t('item-form.quantity.label')}
			bind:value={quantity}
			error={quantityError}
			inputmode="numeric"
		/>
		{#if formError}<p class="form-error" role="alert">{formError}</p>{/if}
	</form>
	{#snippet actions()}
		<Button onclick={() => (open = false)}>{t('item-form.cancel')}</Button>
		<Button variant="primary" type="submit" form="item-form" loading={saving}>
			{t('item-form.save')}
		</Button>
	{/snippet}
</Dialog>

<style>
	.form {
		display: grid;
		gap: calc(var(--space) * 4);
	}

	.form-error {
		margin: 0;
		color: var(--danger);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
	}
</style>
