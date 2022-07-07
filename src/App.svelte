<script lang="ts">
  import './app.css'
  import Item from './lib/Item.svelte'
  import type { Group } from './lib/types'

  let groups: Group[] = [
    { title: 'Rabbit foot water', description: 'Yo', enabled: true, id: 1, nextDate: new Date() },
    { title: 'Rabbit foot water', description: 'Yo', enabled: true, id: 0, nextDate: new Date() },
    { title: 'Rabbit foot water', description: 'Yo', enabled: true, id: 2, nextDate: new Date() },
  ]
</script>

<div class="flex min-h-screen w-full flex-col overflow-y-scroll px-4 py-2">
  <h1 class="mb-2 cursor-default select-none text-center text-2xl font-normal text-white">
    Reminders
  </h1>
  <div class="relative select-none outline-none">
    {#each groups as group (group.id)}
      <Item bind:group />
    {/each}
  </div>
</div>

<style lang="sass">
  :global(body)
    font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial, sans-serif, Apple Color Emoji, Segoe UI Emoji
    font-size: 18px
    color: #ffffff
    overflow: hidden
</style>
