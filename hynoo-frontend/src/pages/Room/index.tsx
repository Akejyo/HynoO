import React from "react";
import { useParams } from "react-router-dom";

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
    <>
      <h1>Room: {name}</h1>
    </>
  );
};

export default Room;
