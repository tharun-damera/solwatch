import { useEffect, useState } from "react";

import "./App.css";
import EmptyState from "./components/EmptyState";
import Navbar from "./components/NavBar";
import SearchBox from "./components/SearchBox";
import Account from "./components/Account";

import TransactionHistory from "./components/TransactionHistory";
import { searchAddress } from "./utils/searchData";
import IndexingUpdates from "./components/IndexingUpdates";

export default function App() {
  let [address, setAddress] = useState("");
  let [loading, setLoading] = useState(false);
  let [account, setAccount] = useState(null);
  let [error, setError] = useState(null);
  let [indexed, setIndexed] = useState(true);
  let [txnsIndexed, settxnsIndexed] = useState(0);

  async function handleSearch(addr) {
    setError(null);
    setLoading(true);
    await searchAddress(addr, setAddress, setIndexed, setAccount);
    setLoading(false);
  }

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const urlAddress = params.get("address");
    if (urlAddress) {
      searchAddress(urlAddress, setAddress, setIndexed, setAccount);
    }
  }, []);

  return (
    <>
      <Navbar />
      <main className="container">
        <SearchBox
          loading={loading}
          address={address}
          onAddress={setAddress}
          onSearch={handleSearch}
        />
        {error && <div className="error">{error}</div>}
        <div class="horizontal-line"></div>

        {!loading && !account && <EmptyState />}

        {!indexed && !error && (
          <IndexingUpdates
            address={address}
            txnsIndexed={txnsIndexed}
            setAccount={setAccount}
            settxnsIndexed={settxnsIndexed}
            setError={setError}
          />
        )}

        {account && <Account data={account} />}

        {account && <div class="horizontal-line"></div>}

        {account && (indexed || txnsIndexed > 0) && (
          <TransactionHistory address={address} />
        )}
      </main>
    </>
  );
}
