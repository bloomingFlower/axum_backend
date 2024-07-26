const eventSource = new EventSource("sse");
const priceElement = document.getElementById("price");
const detailsElement = document.getElementById("details");
const statusElement = document.getElementById("status");
const messagesElement = document.getElementById("messages");

// Update status function
function updateStatus(message, color) {
  statusElement.textContent = message;
  statusElement.style.backgroundColor = color;
  statusElement.style.color = color === "yellow" ? "black" : "white";
}

// Helper function to safely format numbers
function safeFormat(value, decimals = 2) {
  return typeof value === 'number' ? value.toFixed(decimals) : 'Loading...';
}

// Update UI function
function updateUI(data) {
  if (!data) return;

  // Update price
  priceElement.textContent = `$${safeFormat(data.price)}`;

  // Update details
  detailsElement.innerHTML = `
    <p>Last Updated: ${new Date(data.last_updated).toLocaleString()}</p>
    <p>24h High: $${safeFormat(data.high_24h)} | 24h Low: $${safeFormat(data.low_24h)}</p>
    <p>24h Change: $${safeFormat(data.price_change_24h)} (${safeFormat(data.price_change_percentage_24h)}%)</p>
  `;

  // Add to message history
  const messageElement = document.createElement("div");
  messageElement.className = "message";
  messageElement.innerHTML = `
    <h3>Price: $${safeFormat(data.price)}</h3>
    <p>Updated: ${new Date(data.last_updated).toLocaleString()}</p>
    <p>24h Range: $${safeFormat(data.low_24h)} - $${safeFormat(data.high_24h)}</p>
    <p>24h Change: $${safeFormat(data.price_change_24h)} (${safeFormat(data.price_change_percentage_24h)}%)</p>
  `;
  messagesElement.insertBefore(messageElement, messagesElement.firstChild);
}

eventSource.onmessage = function (event) {
  console.log("Message from server:", event.data);
  const message = JSON.parse(event.data);

  if (message.status === "success") {
    updateUI(message.data);
    updateStatus("Connected", "green");
  } else if (message.status === "waiting") {
    updateStatus("Waiting for data...", "yellow");
  } else if (message.status === "error") {
    updateStatus("Error occurred", "red");
  }
};

eventSource.onerror = function (error) {
  console.error("EventSource failed:", error);
  updateStatus("Connection error. Trying to reconnect...", "red");
};

eventSource.onopen = function () {
  console.log("EventSource connection opened");
  updateStatus("Connected", "green");
};

// Close the connection when the page is left
window.addEventListener("beforeunload", () => {
  eventSource.close();
});