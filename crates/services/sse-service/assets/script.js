const eventSource = new EventSource("sse");
const priceElement = document.getElementById("price");
const detailsElement = document.getElementById("details");
const statusElement = document.getElementById("status");
const messagesElement = document.getElementById("messages");

function updateStatus(message, color) {
  statusElement.textContent = message;
  statusElement.style.backgroundColor = color;
  statusElement.style.color = color === "yellow" ? "black" : "white";
}

eventSource.onmessage = function (event) {
  console.log("Message from server:", event.data);
  const data = JSON.parse(event.data);

  // Update price
  priceElement.textContent = `$${data.price.toFixed(2)}`;

  // Update details
  detailsElement.innerHTML = `
        <p>Last Updated: ${new Date(data.last_updated).toLocaleString()}</p>
        <p>24h High: $${data.high_24h.toFixed(
          2
        )} | 24h Low: $${data.low_24h.toFixed(2)}</p>
        <p>24h Change: $${data.price_change_24h.toFixed(
          2
        )} (${data.price_change_percentage_24h.toFixed(2)}%)</p>
    `;

  // Add to message history
  const messageElement = document.createElement("div");
  messageElement.className = "message";
  messageElement.innerHTML = `
        <h3>Price: $${data.price.toFixed(2)}</h3>
        <p>Updated: ${new Date(data.last_updated).toLocaleString()}</p>
        <p>24h Range: $${data.low_24h.toFixed(2)} - $${data.high_24h.toFixed(
    2
  )}</p>
        <p>24h Change: $${data.price_change_24h.toFixed(
          2
        )} (${data.price_change_percentage_24h.toFixed(2)}%)</p>
    `;
  messagesElement.insertBefore(messageElement, messagesElement.firstChild);

  // Update status
  updateStatus("Connected", "green");
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
