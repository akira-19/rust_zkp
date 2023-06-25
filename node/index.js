const net = require('net');
const client = new net.Socket();

const serverAddress = '/tmp/socket_file';

// ソケット接続のハンドラー
client.connect(serverAddress, function () {
  console.log('Connected');
  client.write('Hello, rust server!');
});

// データ受信のハンドラー
client.on('data', function (data) {
  console.log('Received: ' + data);
  client.destroy(); // 接続を閉じる
});

// 接続が閉じられたときのハンドラー
client.on('close', function () {
  console.log('Connection closed');
});
