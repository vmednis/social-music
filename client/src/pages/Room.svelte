<script>
  import {onDestroy} from "svelte";
  import Player from "/src/pages/room/Player.svelte"
  import Chat from "/src/pages/room/Chat.svelte";
  import {socket} from "./room/socket.js";

  export let roomId = "";

  socket.init(roomId);
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