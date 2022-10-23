<script>
  let message;
  let messages = [];

  function writeMessage(message) {
    messages.push(message);
    messages = messages;
  }

  function onChatBoxKeyDown(event) {
    if(event.key == "Enter") {
      writeMessage(message);
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
        <p>{message}</p>
      {/each}
    </div>
  </div>
  <input id="chatbox" type="text" class="border border-solid grow-0" placeholder="Send a message..." bind:value={message} on:keydown={onChatBoxKeyDown}/>
</div>

<style>
</style>