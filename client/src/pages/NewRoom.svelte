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

<ul>
  {#each errors as error}
    <li>{error}</li>
  {/each}
</ul>
<form>
  <label for="room_id">Room Id:</label>
  <input type="text" id="room_id" class="border" bind:value={room.id}><br>
  <label for="room_name">Room Name:</label>
  <input type="text" id="room_name" class="border" bind:value={room.title}><br>
</form>
{#if ready}
  <button class="text-blue-600" on:click="{onCreateRoom}">Create Room</button>
{/if}
