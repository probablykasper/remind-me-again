<script lang="ts">
  import { cubicOut } from 'svelte/easing'
  import { crossfade } from 'svelte/transition'
  import { flip } from 'svelte/animate'
  import './app.css'
  import Item from './lib/Item.svelte'
  import New from './lib/New.svelte'
  import type { Group } from './lib/types'

  let lastId = 0
  function newId() {
    return lastId++
  }
  let groups: Group[] = [
    {
      title: 'Rabbit stuff',
      description: 'Yo',
      enabled: true,
      id: newId(),
      nextDate: new Date(),
      repeat: 'never',
    },
    {
      title: 'Rabbit stuff',
      description: 'Yo',
      enabled: true,
      id: newId(),
      nextDate: new Date(),
      repeat: 'never',
    },
    {
      title: 'Rabbit stuff',
      description: 'Yo',
      enabled: true,
      id: newId(),
      nextDate: new Date(),
      repeat: 'never',
    },
  ]

  const [send, receive] = crossfade({
    fallback(node) {
      const style = getComputedStyle(node)
      const transform = style.transform === 'none' ? '' : style.transform

      return {
        duration: 400,
        easing: cubicOut,
        css: (t) => `
  				transform: ${transform} scale(${1 + (t - 1) * 0.2});
  				opacity: ${t}
  			`,
      }
    },
  })
</script>

<div class="flex min-h-screen w-full flex-col overflow-y-scroll px-4 py-2">
  <h1 class="mb-2 cursor-default select-none text-center text-2xl font-normal text-white">
    Reminders
  </h1>
  <div class="relative select-none outline-none">
    <New
      onCreate={(group) => {
        group.id = newId()
        groups = [...groups, group]
      }}
    />
    {#each groups as group (group.id)}
      <div
        in:receive={{ key: group.id, duration: 400 }}
        out:send={{ key: group.id, duration: 400 }}
        animate:flip={{ duration: 400 }}
      >
        <Item
          bind:group
          onDelete={() => {
            groups = groups.filter((g) => g.id !== group.id)
          }}
        />
      </div>
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
