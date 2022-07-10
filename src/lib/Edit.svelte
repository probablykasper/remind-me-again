<script lang="ts">
  import { DateInput } from 'date-picker-svelte'
  import { slide, fly } from 'svelte/transition'
  import type { Group } from './types'

  export let group: Group

  export let onSave: () => void
  export let onCancel = () => {
    /* noop */
  }

  let nextDate = group.next_date === null ? null : new Date(group.next_date)
  $: group.next_date = nextDate === null ? null : nextDate.getTime()
</script>

<div in:slide={{ duration: 200 }} out:slide={{ duration: 300 }}>
  <div in:fly={{ duration: 300, y: -20 }} out:fly={{ duration: 200, y: -20 }}>
    <div class="mt-2">
      <DateInput
        bind:value={nextDate}
        closeOnSelection={true}
        --date-picker-background="#031212"
        --date-picker-foreground="#f7f7f7"
        --date-input-width="100%"
        --date-picker-highlight-border="hsl(183, 98%, 49%)"
        --date-picker-highlight-shadow="hsla(183, 98%, 49%, 50%)"
        --date-picker-selected-color="hsl(183, 100%, 85%)"
        --date-picker-selected-background="hsla(183, 98%, 49%, 20%)"
      />
    </div>
    <span class="text-sm">Repeat</span>
    <select
      class="mt-2 rounded border-none bg-white bg-opacity-10 py-1 pl-3 pr-8 text-sm outline-none focus:focus:ring-2 focus:ring-[#02ebf7]"
      bind:value={group.repeat}
    >
      <option value={'never'}>Never</option>
      <option value={'daily'}>Daily</option>
    </select>
    <div class="flex w-full">
      <button
        type="button"
        class="mt-2 mr-2 w-full cursor-default rounded-sm bg-[#576f70] px-2 py-1 text-sm"
        on:click|stopPropagation={() => onCancel()}>Cancel</button
      >
      <button
        type="submit"
        class="mt-2 w-full cursor-default rounded-sm bg-[#31898c] px-2 py-1 text-sm"
        on:click|stopPropagation={() => onSave()}>Save</button
      >
    </div>
  </div>
</div>
