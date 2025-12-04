import { useEffect, useState } from "react";

import EmptyState from "./components/EmptyState";
import Navbar from "./components/NavBar";
import SearchBox from "./components/SearchBox";
import Account from "./components/Account";

import TransactionHistory from "./components/TransactionHistory";
import IndexerStats from "./components/IndexerStats";
import Transaction from "./components/Transaction";

import { accountIndexStatus } from "./api/api";

export default function App() {
  let [loading, setLoading] = useState(false);
  let [error, setError] = useState(null);

  let [address, setAddress] = useState("");
  let [indexed, setIndexed] = useState(null);
  let [account, setAccount] = useState(null);
  let [txnsIndexed, setTxnsIndexed] = useState(0);
  let [detailedTxn, setDetailedTxn] = useState(null);

  async function handleSearch(addr) {
    setError(null);
    setIndexed(null);
    setLoading(true);

    setAddress(addr);
    window.history.replaceState({}, "", `?address=${addr}`);

    let result = await accountIndexStatus(addr);
    if (result.indexed) {
      setIndexed(true);
    } else {
      setIndexed(false);
    }
    setLoading(false);
  }

  useEffect(() => {
    function pageRefresh() {
      const params = new URLSearchParams(window.location.search);
      const urlAddress = params.get("address");
      if (urlAddress) {
        handleSearch(urlAddress);
      }
    }
    pageRefresh();
  }, []);

  return (
    <>
      <Navbar />
      <main className="container">
        <SearchBox
          loading={loading}
          setAddress={setAddress}
          onSearch={handleSearch}
        />
        {error && <div className="error">{error}</div>}
        <div className="horizontal-line"></div>

        {indexed == null && <EmptyState />}

        {indexed != null && (
          <IndexerStats
            address={address}
            indexed={indexed}
            setAccount={setAccount}
            setTxnsIndexed={setTxnsIndexed}
            setError={setError}
          />
        )}

        {indexed && (
          <Account
            address={address}
            account={account}
            setAccount={setAccount}
            setError={setError}
          />
        )}

        {account && (indexed || txnsIndexed > 0) && (
          <TransactionHistory
            address={address}
            account={account}
            setDetailedTxn={setDetailedTxn}
            setError={setError}
          />
        )}

        {detailedTxn && (
          <Transaction
            address={address}
            signature={detailedTxn}
            error={error}
            setError={setError}
            onClose={() => setDetailedTxn(null)}
          />
        )}
      </main>
    </>
  );
}
