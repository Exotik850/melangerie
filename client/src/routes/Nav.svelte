<!-- <script lang="ts">
  import { toast } from "svelte-french-toast";
  import { sendMessage, timedIn, uname } from "$lib/stores";
  import { host } from "$lib";
  import Popup from "../components/Popup.svelte";

  let timingIn = false;
  let note: string | null = null;

  async function sendReport() {
    let issue = prompt("Enter the issue you are facing:");
    if (!$uname || !issue) {
      toast.error("Cannot send an empty report!");
      return;
    }
    let res = await fetch(host + `/report`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: $uname, issue }),
    });
    if (res.status === 200) {
      toast.success("Issue reported successfully!");
    } else {
      toast.error("Failed to report issue! Please try again later.");
    }
  }

  function toggle() {
    timingIn = !timingIn;
    sendMessage({
      action: "CheckTime",
    });
  }

  function time_inout(e: MouseEvent) {
    e.preventDefault();
    if (!note) {
      note = null;
    }
    sendMessage({
      action: $timedIn ? "TimeOut" : "TimeIn",
      data: note,
    });
    timingIn = false;
    sendMessage({
      action: "CheckTime",
    });
    setTimeout(() => {
      toast.success(
        $timedIn ? "Timed in successfully!" : "Timed out successfully!"
      );
    }, 75);
    note = null;
  }
</script>

<Popup bind:open={timingIn}>
  {#if $timedIn}
    <h1>Time Out</h1>
  {:else}
    <h1>Time In</h1>
  {/if}
  <p>{new Date().toLocaleString()}</p>
  <form>
    <input type="text" bind:value={note} placeholder="Note" />
    <button on:click={time_inout}>Submit</button>
  </form>
</Popup>

<div class="nav">
  <h1>Melangerie</h1>
  <div class="buttons gray">
    <button on:click={sendReport}>Report Issue</button>
    <div class="buttons">
      {#if $timedIn}
        <p>Timed In</p>
      {:else}
        <p>Timed Out</p>
      {/if}
      <div
        style="width: 10px; height: 10px; border-radius: 50%; background-color: {$timedIn?'green':'red'};"
      ></div>
    </div>
    <button on:click={() => (timingIn = !timingIn)}>Time In/Out</button>
    <button on:click={() => (window.location.href = "/")}>Logout</button>
  </div>
</div> -->

<!-- <style lang="scss">
  .nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background-color: #767575;
    color: white;
    height: 5vh;
    button {
      padding: 0.5rem 1rem;
      border: none;
      background-color: #999898;
      color: white;
      cursor: pointer;
    }
    h1 {
      font-family: "Trebuchet MS", "Lucida Sans Unicode", "Lucida Grande",
        "Lucida Sans", Arial, sans-serif;
    }
  }

  .buttons {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 1rem;
  }

  .gray {
    background-color: transparentize(#0a0a0a, 0.8);
    border-radius: 1rem;
    padding: 0.5rem;
  }
</style> -->

<script lang="ts">
  import { toast } from "svelte-french-toast";
  import { sendMessage, timedIn, uname } from "$lib/stores";
  import { host } from "$lib";
  import Popup from "../components/Popup.svelte";

  let timingIn = false;
  let note: string | null = null;

  async function sendReport() {
    let issue = prompt("Enter the issue you are facing:");
    if (!$uname || !issue) {
      toast.error("Cannot send an empty report!");
      return;
    }
    let res = await fetch(host + `/report`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: $uname, issue }),
    });
    if (res.status === 200) {
      toast.success("Issue reported successfully!");
    } else {
      toast.error("Failed to report issue! Please try again later.");
    }
  }

  function time_inout(e: MouseEvent) {
    e.preventDefault();
    if (!note) {
      note = null;
    }
    sendMessage({
      action: "TimingAction",
      data: {
        action: $timedIn ? "TimeOut" : "TimeIn",
        note,
      },
    });
    timingIn = false;
    sendMessage({
      action: "CheckTime",
    });
    setTimeout(() => {
      toast.success(
        $timedIn ? "Timed in successfully!" : "Timed out successfully!"
      );
    }, 75);
    note = null;
  }
</script>
<svelte:head>
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/5.15.4/css/all.min.css">
</svelte:head>

<Popup bind:open={timingIn}>
  {#if $timedIn}
    <h1>Time Out</h1>
  {:else}
    <h1>Time In</h1>
  {/if}
  <p>{new Date().toLocaleString()}</p>
  <form>
    <input type="text" bind:value={note} placeholder="Note" />
    <button on:click={time_inout}>Submit</button>
  </form>
</Popup>
<nav class="navbar">
  <div class="navbar-brand">
    <h1>Melangerie</h1>
  </div>
  <div class="navbar-buttons">
    <button class="navbar-button" on:click={sendReport}>
      <i class="fas fa-exclamation-circle"></i>
      <span>Report Issue</span>
    </button>
    <div class="navbar-timein">
      <button class="navbar-button" on:click={() => (timingIn = !timingIn)}>
        <i class="fas fa-clock"></i>
        <span>{$timedIn ? 'Timed In' : 'Timed Out'}</span>
        <div class="timein-indicator" class:timed-in={$timedIn}></div>
      </button>
    </div>
    <button class="navbar-button" on:click={() => (window.location.href = "/")}>
      <i class="fas fa-sign-out-alt"></i>
      <span>Logout</span>
    </button>
  </div>
</nav>
<style lang="scss">
  .navbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background-color: #f8f9fa;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    height: 5vh;
  }

  .navbar-brand {
    h1 {
      font-family: "Trebuchet MS", "Lucida Sans Unicode", "Lucida Grande",
        "Lucida Sans", Arial, sans-serif;
      color: #343a40;
      margin: 0;
    }
  }

  .navbar-buttons {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .navbar-button {
    display: flex;
    align-items: center;
    padding: 0.5rem 1rem;
    border: none;
    background-color: #e9ecef;
    color: #343a40;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.3s ease;

    &:hover {
      background-color: #dee2e6;
    }

    i {
      margin-right: 0.5rem;
    }
  }

  .navbar-timein {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .timein-indicator {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background-color: #dc3545;
    transition: background-color 0.3s ease;
    margin-left: 0.5rem;

    &.timed-in {
      background-color: #28a745;
    }
  }
</style>