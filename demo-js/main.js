const express = require('express');
const ws = require('ws');
const MediaServer = require('medooze-media-server');
const SemanticSDP = require("semantic-sdp");

MediaServer.enableLog(true);
MediaServer.enableDebug(true);
MediaServer.enableUltraDebug(false);

const wsServer = new ws.Server({ noServer: true });

wsServer.on('connection', socket => {
    socket.on('message', message => {
        const parsed = JSON.parse(message);
        console.log(parsed);

        if (parsed.type !== 'offer') {
            console.warn('unknown message:', parsed.type);
            return;
        }

        const endpoint = MediaServer.createEndpoint('127.0.0.1');

        socket.on('close', () => {
            endpoint.stop();
        });

        const offer = SemanticSDP.SDPInfo.parse(parsed.sdp);

        const transport = endpoint.createTransport({
            dtls: offer.getDTLS(),
            ice: offer.getICE(),
        });

        transport.setRemoteProperties({
            audio: offer.getMedia("audio"),
            video: offer.getMedia("video"),
        });

        const dtls = transport.getLocalDTLSInfo();
        const ice = transport.getLocalICEInfo();

        const candidates = endpoint.getLocalCandidates();

        const capabilities = MediaServer.getDefaultCapabilities();

        const answer = offer.answer({
            dtls,
            ice,
            candidates,
            capabilities,
        });

        transport.setLocalProperties({
            audio: answer.getMedia("audio"),
            video: answer.getMedia("video"),
        });

        console.dir(offer, { depth: null });

        const offered = offer.getStreams().values().next().value;

        console.dir(offered, { depth: null });

        const incomingStream = transport.createIncomingStream(offered);

        socket.send(JSON.stringify({
            type: 'answer',
            sdp: answer.toString(),
        }));
    });
});

const app = express();

app.use(express.static(__dirname + '/../demo/resources/'));

const server = app.listen(8080);

server.on('upgrade', (request, socket, head) => {
    wsServer.handleUpgrade(request, socket, head, socket => {
        wsServer.emit('connection', socket, request);
    });
});
