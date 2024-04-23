import React, { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import MessageBox from "../../components/MessageBox";

const Room = () => {
  const { name: roomName } = useParams();
  const [username, setUsername] = useState<string | null>();
  const [messages, setMessages] = useState<
    { username: string; content: string }[]
  >([]);

  //Get room name
  if (roomName === undefined) return null;
  let name = roomName;
  if (name[0] === ">") {
    name = name.slice(1);
  } else return null;

  //Get username
  useEffect(() => {
    const username_tmp = window.prompt("Please enter your username");
    setUsername(username_tmp);
  }, []);

  //Connect to WebSocket
  useEffect(() => {
    const ws = new WebSocket("ws://localhost:3030/chat");
    ws.onopen = () => {
      console.log("connected");
      ws.send(`${username}`);
    };
    ws.onmessage = (e) => {
      const newMessage = JSON.parse(e.data);
      setMessages((oldMessages) => [...oldMessages, newMessage]);
    };
  }, [username]);

  //Send message
  const sendMessage = () => {
    const input = document.getElementById("message") as HTMLInputElement;
    const content = input.value;
    if (content === "") return;
    const newMessage = { username: "User_tmp", content };
    setMessages((oldMessages) => [...oldMessages, newMessage]);
    input.value = "";
  };

  return (
    <div className="room">
      <h2>Room: {name}</h2>
      <div className="partingLine"></div>
      <div style={{ display: "flex", justifyContent: "center" }}>
        <div className="roomBox">
          {messages.map((message, index) => (
            <MessageBox
              key={index}
              username={message.username}
              content={message.content}
            />
          ))}
        </div>
      </div>
      <div className="input">
        <textarea
          id="message"
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              sendMessage();
            }
          }}
        />
        <button onClick={sendMessage}>Send</button>
      </div>
    </div>
  );
};

export default Room;
