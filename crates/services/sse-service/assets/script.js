const eventSource = new EventSource("sse");
const priceElement = document.getElementById("price");
const countdownElement = document.getElementById("countdown");
const messagesElement = document.getElementById("messages");

eventSource.onmessage = function (event) {
  console.log("Message from server:", event.data);
  const data = JSON.parse(event.data);

  // Update price
  priceElement.textContent = `Current Bitcoin Price: $${data.price.toFixed(2)}`;

  // Update countdown
  countdownElement.textContent = `Next update in: ${data.countdown} seconds`;

  // Add to message history
  const messageElement = document.createElement("p");
  messageElement.textContent = `Price: $${data.price.toFixed(2)}, Countdown: ${
    data.countdown
  }s`;
  messagesElement.insertBefore(messageElement, messagesElement.firstChild);
};

eventSource.onerror = function (error) {
  console.error("EventSource failed:", error);
  const errorElement = document.createElement("p");
  errorElement.textContent = "Connection error. Trying to reconnect...";
  errorElement.style.color = "red";
  messagesElement.insertBefore(errorElement, messagesElement.firstChild);
};

eventSource.onopen = function () {
  console.log("EventSource connection opened");
  const openElement = document.createElement("p");
  openElement.textContent = "Connection established.";
  openElement.style.color = "green";
  messagesElement.insertBefore(openElement, messagesElement.firstChild);
};

// Close the connection when the page is left
window.addEventListener("beforeunload", () => {
  eventSource.close();
});
