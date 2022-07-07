<script lang="ts">
  import './app.css'

  type Group = {
    title: string
    description: string
    enabled: boolean
  }

  let groups: Group[] = [
    { title: 'Rabbit foot water', description: 'Yo', enabled: true },
    { title: 'Rabbit foot water', description: 'Yo', enabled: false },
  ]
</script>

<div class="mx-6 my-2">
  <h1 class="mb-2 cursor-default select-none text-2xl font-normal text-white">Reminders</h1>
  {#each groups as group}
    <div
      class="my-6 flex items-center rounded-lg border border-white border-opacity-[1%] bg-[#131214] px-4 py-4"
    >
      <div>
        <p class="text-sm">{group.title}</p>
        <p class="text-xs text-white text-opacity-75">{group.description}</p>
      </div>
      <div
        class="ml-auto rounded-md bg-[#16261E] bg-opacity-70 p-2.5 text-[#3ee570] transition-colors ease-out hover:bg-opacity-100"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="currentColor"
          ><path
            d="M14.078 7.061l2.861 2.862-10.799 10.798-3.584.723.724-3.585 10.798-10.798zm0-2.829l-12.64 12.64-1.438 7.128 7.127-1.438 12.642-12.64-5.691-5.69zm7.105 4.277l2.817-2.82-5.691-5.689-2.816 2.817 5.69 5.692z"
          /></svg
        >
      </div>
    </div>
  {/each}
</div>

<style lang="sass">
  :global(body)
    font-family: -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial, sans-serif, Apple Color Emoji, Segoe UI Emoji
    font-size: 18px
    background-color: #000000
    color: #ffffff
</style>
