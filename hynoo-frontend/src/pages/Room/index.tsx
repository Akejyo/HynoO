import React from "react";
import { useParams } from "react-router-dom";
import MessageBox from "../../components/MessageBox";

const Room = () => {
  const { name: roomName } = useParams();
  if (roomName === undefined) return null;

  let name = roomName;
  if (name[0] === ">") {
    name = name.slice(1);
  } else {
    return null;
  }

  return (
    <div className="room">
      <h2>Room: {name}</h2>
      <div className="partingLine"></div>
      <div style={{ display: "flex", justifyContent: "center" }}>
        <div className="roomBox">
          <MessageBox username="User1" content="Hello!" />
          <MessageBox username="User2" content="你好!" />
        </div>
      </div>

      {/* <div className="input">
        <input type="text" />
        <button>Send</button>
      </div> */}
    </div>
  );
};

export default Room;
