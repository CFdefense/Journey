import React from "react";
import { MessageBox } from "react-chat-elements";
import { Input } from 'react-chat-elements'
import { Button } from "react-chat-elements";

import "react-chat-elements/dist/main.css";

export default function ChatBox() {
  return (
    <div style={{ padding: "20px" }}>
      <MessageBox
        {...({
          position: "left",
          type: "text",
          text: "Here is a text type message box",
        } as any)}
      />

      <Input
        {...({
          placeholder: "Type here...",
          multiline: true,
        } as any)}
      />

      <Button
        {...({
          text: "Send",
          onClick: () => alert("Sending..."),
          title: "Send",
        } as any)}
      />
    </div>
  );
}
