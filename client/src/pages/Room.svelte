<script>
  import {onDestroy} from "svelte";
  import Player from "/src/pages/room/Player.svelte"
  import Chat from "/src/pages/room/Chat.svelte";
  import {socket} from "./room/socket.js";

  export let room_id = "";
  console.log(room_id);

  socket.init();
  onDestroy(() => {
    socket.close();
  })
</script>

<div class="flex flex-col h-full max-h-full">
  <Player></Player>
  <Chat></Chat>
</div>

<style>
</style>