import React from "react";
//接收username和content
interface MessageBoxProps {
  username: string;
  content: string;
}
const MessageBox: React.FC<MessageBoxProps> = ({ username, content }) => {
  return (
    <div className="message">
      <div className="name">{username}</div>
      <div className="content">{content}</div>
    </div>
  );
};
export default MessageBox;
