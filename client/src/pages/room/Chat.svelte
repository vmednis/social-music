<script>
  import { socket } from "./socket.js";
  let message;

  function onChatBoxKeyDown(event) {
    if(event.key == "Enter") {
      const queue = message.match(/\/queue ([^ ]+)/);
      if(queue) {
        socket.sendQueueSong(queue[1]);
      }

      const join = message.match(/\/join/);
      if(join) {
        socket.sendJoinQueue();
      }

      socket.sendChatMessage(message);
      event.preventDefault();
      event.target.value = "";
    }
  }
</script>

<div class="flex flex-col h-full max-h-full grow bg-slate-50">
  <div class="flex-1 flex flex-col overflow-y-scroll">
    <div class="grow" />
    <div class="grow-0 flex flex-col">
      {#each $socket.messages as message}
        <div class="border-t-2 border-gray-200">
          <p><b>{message.from}:&nbsp;</b>{message.message}</p>
        </div>
      {/each}
    </div>
  </div>
  <input id="chatbox" type="text" class="border border-solid grow-0" placeholder="Send a message..." bind:value={message} on:keydown={onChatBoxKeyDown} disabled={!$socket.ready} autocomplete="off"/>
</div>

<style>
</style>