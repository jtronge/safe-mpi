# Address set up for two nodes
export SERVER=$(hostname)
export SERVER_IP=$(python3 -c "import socket; print(socket.gethostbyname('$SERVER'))")
