# Advanced Programming Rust 
by Andriyo Averill Fahrezi, NPM of 2306172325

## Module 10 - Asynchronous Programming - Chat App Web Socket

### Original Code

![Screenshot 2025-05-22 212431](https://github.com/user-attachments/assets/ddd655e8-2f74-4bea-81a9-827094240ac9)

### Be Creative!

![image](https://github.com/user-attachments/assets/4644902c-1d43-461e-b7c1-d32c92e4cf5c)
![image](https://github.com/user-attachments/assets/bd50c5c4-b7de-4df6-8e91-597b69c653a3)
![image](https://github.com/user-attachments/assets/21bceb0f-f56e-4323-b63b-740dfd1d8fb1)
![image](https://github.com/user-attachments/assets/8e329fb3-da45-4641-87ed-2dfecb575f56)
![image](https://github.com/user-attachments/assets/54097bdc-96c1-42ac-b127-446fa213fb7d)

Based on the image above, I've implemented several new features to enhance the chat experience and to show how creative I am based on this tutorial: 

- Emoji Button & Picker: A new button next to the chat input now provides an emoji picker. Users can easily click an emoji to insert it directly into their message. This is handled by `Msg::ToggleEmojiPicker` and `Msg::AddEmoji` in `Chat`.

- Reaction Button: Each chat message now includes reaction buttons (e.g., 👍, 😂, ❤️). Users can react to messages, with their chosen reaction prominently displayed below the message. Clicking a reaction sends an event via WebSocket, and the UI updates to show who reacted with which emoji. See `Msg::ReactToMessage` and the reaction rendering in `Chat`.

- Online Users Counter: The sidebar now features an online users counter, displaying the current number of active users. The user list is dynamically updated via WebSocket messages, and the count is reflected in the sidebar's rendering within `Chat`.

- Theme Switcher (Background Color Button): Users can now personalize their chat experience with a theme switcher. Buttons allow switching between various chat themes (Classic, Midnight, Sky, Forest), which instantly change the background and message bubble colors. Clicking a theme button updates the `theme` state, which in turn modifies CSS classes for the chat UI. Relevant code can be found in the `Theme` enum and the theme switcher buttons within `Chat`.