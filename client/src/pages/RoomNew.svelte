<script>
	import { navigate } from "svelte-navigator";

  let ready = true;
  let room = {
    id: "",
    title: "",
  };
  let errors = [];

  function onCreateRoom(event) {
    event.preventDefault();
    ready = false;
    fetch("/room/new", {
      method: "POST",
      headers: {
        "Accept": "application/json",
        "Content-Type": "application/json"
      },
      body: JSON.stringify(room)
    }).then((response) => response.text()).then((body) => {
      errors = JSON.parse(body);
      if(errors.length == 0) {
        navigate("/room/" + room.id);
      }
      ready = true;
    });
  }
</script>

<div class="w-full h-full grow bg-slate-50 flex justify-center">
  <div class="max-w-md w-full flex flex-col justify-center">
    <div class="w-full bg-slate-100 p-4 border-slate-200 rounded border">
      <h1 class="text-2xl">Create Room</h1>
      {#if errors.length > 0}
        <h2 class="text-red-700 text-lg">Errors:</h2>
        <ul class="list-disc list-inside text-red-700">
          {#each errors as error}
            <li>{error}</li>
          {/each}
        </ul>
      {/if}
      <form>
        <label for="room_id" class="font-medium">Room id:</label><br/>
        <input type="text" id="room_id" class="border-slate-200 border rounded w-full p-1 mb-1" bind:value={room.id}><br>
        <label for="room_name" class="font-medium">Room name:</label><br/>
        <input type="text" id="room_name" class="border-slate-200 border rounded w-full p-1 mb-4" bind:value={room.title}><br>
      </form>
      {#if ready}
        <button class="p-1 w-full border-amber-500 border rounded bg-amber-400 hover:bg-amber-500" on:click="{onCreateRoom}">Create Room</button>
      {/if}
    </div>
  </div>
</div>