<script>
  import {onDestroy} from "svelte";
  import {socket} from "./room/socket.js";
  import Player from "/src/pages/room/Player.svelte"
  import Chat from "/src/pages/room/Chat.svelte";
  import PanelRight from "/src/pages/room/PanelRight.svelte";
  import PanelLeft from "/src/pages/room/PanelLeft.svelte";

  export let roomId = "";

  socket.init(roomId);
  onDestroy(() => {
    socket.close();
  })
</script>

<div class="flex flex-col h-full max-h-full">
  <Player></Player>
  <div class="flex flex-row h-full max-h-full min-h-0 grow">
    <PanelLeft/>
    <Chat></Chat>
    <PanelRight/>
  </div>
</div>