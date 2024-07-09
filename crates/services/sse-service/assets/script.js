const eventSource = new EventSource('sse');

eventSource.onmessage = function(event) {
    console.log('Message from server:', event.data);
    // 받은 데이터를 화면에 표시
    const messageElement = document.createElement('p');
    messageElement.textContent = event.data;
    document.body.appendChild(messageElement);
};

eventSource.onerror = function(error) {
    console.error('EventSource failed:', error);
};

eventSource.onopen = function() {
    console.log('EventSource connection opened');
};

// 페이지를 떠날 때 연결 종료
window.addEventListener('beforeunload', () => {
    eventSource.close();
});