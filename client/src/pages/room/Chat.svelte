<script>
  let message;
  let messages = [];

  const socket = new WebSocket("ws://127.0.0.1:3030/chat");
  let socket_ready = false;
  socket.addEventListener('open', (event) => {
    socket_ready = true;
  })
  socket.addEventListener('message', (event) => {
    let message = JSON.parse(event.data);
    writeMessage(message);
  })

  function sendMessage(message) {
    if(socket_ready) {
      socket.send(message);
    }
  }

  function writeMessage(message) {
    messages.push(message);
    messages = messages;
  }

  function onChatBoxKeyDown(event) {
    if(event.key == "Enter") {
      sendMessage(message);
      event.preventDefault();
      event.target.value = "";
    }
  }
</script>

<div class="flex flex-col h-full max-h-full">
  <div class="flex-1 flex flex-col overflow-y-scroll">
    <div class="grow" />
    <div class="grow-0 flex flex-col">
      {#each messages as message}
        <div class="border border-t-2">
          <p><b>{message.from}:&nbsp;</b>{message.message}</p>
        </div>
      {/each}
    </div>
  </div>
  <input id="chatbox" type="text" class="border border-solid grow-0" placeholder="Send a message..." bind:value={message} on:keydown={onChatBoxKeyDown} disabled={!socket_ready}/>
</div>

<style>
</style>