<script lang="ts">
  import { usersStore } from "$lib/stores";
import Popup from "../components/Popup.svelte";
export let callback: (user: string) => void;
export let open = false;

</script>

<Popup bind:open>
  <h1>Choose a user to add: </h1>
  <div class="user-container"> 
    {#each $usersStore as user} 
      <button class="user-button" on:click={() => callback(user)}>{user}</button>
    {/each} 
  </div>
</Popup>

<style lang="scss">
  .user-button {
    margin: 5px;
    padding: 5px;
    border: 1px solid black;
    border-radius: 5px;
    background-color: white;
    cursor: pointer;
    &:hover {
      background-color: #f0f0f0;
    }
    width: 100%;
  }

  .user-container {
    display: flex;
    flex-wrap: wrap;
    overflow: auto;
    height: 100%;
  }
</style>