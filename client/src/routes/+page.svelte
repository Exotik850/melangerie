<script lang="ts">
  import ChatRoom from "./ChatRoom.svelte";
  import { token_store } from "$lib/stores";
  import Modal from "../components/Modal.svelte";
</script>

<main>
  {#if $token_store}
    <ChatRoom />
  {:else}
    <Modal />
  {/if}
</main>

<style lang="scss">
</style>
