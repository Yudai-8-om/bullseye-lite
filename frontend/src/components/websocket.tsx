import { useEffect, useRef, useState } from "react";

const RENDERING_INTERVAL = 5000; // 5 seconds
function WebSocketComponent() {
  const priceRef = useRef<string | null>(null);
  const [price, setPrice] = useState<string | null>(null);
  const [updated, setUpdated] = useState<boolean>(false);
  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    const socket = new WebSocket("ws://192.168.1.12/ws");
    socket.onopen = () => {
      console.log("Connected");
      intervalRef.current = setInterval(() => {
        if (priceRef.current) {
          setPrice(priceRef.current);
        }
      }, RENDERING_INTERVAL);
      setUpdated(true);
    };

    socket.onclose = () => {
      console.log("Disconnected");
      setUpdated(false);
    };

    socket.onmessage = (event) => {
      priceRef.current = event.data;
      setPrice((prevPrice) => (!prevPrice ? event.data : prevPrice));
    };

    socket.onerror = (event) => {
      console.error(`Error: ${event}`);
      setUpdated(false);
    };
    return () => {
      socket.close();
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  return (
    <div>
      <h1 className="text-xl font-bold mb-4">WebSocket Test</h1>
      <h2>{price}</h2>
      {updated ? <div>🟢active</div> : <div>🔴inactive</div>}
    </div>
  );
}
export default WebSocketComponent;
