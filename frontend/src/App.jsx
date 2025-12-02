import { useEffect, useState } from "react";

import EmptyState from "./components/EmptyState";
import Navbar from "./components/NavBar";
import SearchBox from "./components/SearchBox";
import Account from "./components/Account";

import TransactionHistory from "./components/TransactionHistory";
import { searchAddress } from "./utils/searchData";
import IndexingUpdates from "./components/IndexingUpdates";
import Transaction from "./components/Transaction";

export default function App() {
  let [address, setAddress] = useState("");
  let [loading, setLoading] = useState(false);
  let [account, setAccount] = useState(null);
  let [error, setError] = useState(null);
  let [indexed, setIndexed] = useState(true);
  let [signaturesIndexed, setSignaturesIndexed] = useState(0);
  let [txnsIndexed, setTxnsIndexed] = useState(0);
  let [detailedTxn, setDetailedTxn] = useState(null);

  async function handleSearch(addr) {
    setError(null);
    setIndexed(true);
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
          setAddress={setAddress}
          onSearch={handleSearch}
        />
        {error && <div className="error">{error}</div>}
        <div className="horizontal-line"></div>

        {!loading && !account && <EmptyState />}

        {!indexed && !error && (
          <>
            <IndexingUpdates
              address={address}
              signaturesIndexed={signaturesIndexed}
              txnsIndexed={txnsIndexed}
              setAccount={setAccount}
              setSignaturesIndexed={setSignaturesIndexed}
              setTxnsIndexed={setTxnsIndexed}
              setError={setError}
            />
            <div className="horizontal-line"></div>
          </>
        )}

        {account && (
          <>
            <Account data={account} />
            <div className="horizontal-line"></div>
          </>
        )}

        {account && (indexed || signaturesIndexed > 0) && (
          <TransactionHistory
            address={address}
            account={account}
            setDetailedTxn={setDetailedTxn}
          />
        )}

        {detailedTxn && (
          <Transaction
            address={address}
            signature={detailedTxn}
            onClose={() => setDetailedTxn(null)}
          />
        )}
      </main>
    </>
  );
}
