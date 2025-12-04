import { useState } from "react";

export default function SearchBox({ loading, setAddress, onSearch }) {
  let [addr, setAddr] = useState("");

  function onSubmit(e) {
    e.preventDefault();
    setAddress(addr);
    onSearch(addr);
  }

  return (
    <form className="search-box" onSubmit={onSubmit}>
      <input
        id="search-input"
        type="text"
        placeholder="Enter a Solana Wallet Address"
        value={addr}
        onChange={(e) => setAddr(e.target.value.trim())}
      />
      <button className="sol-gradient-btn" type="submit" disabled={loading}>
        Search
      </button>
    </form>
  );
}
