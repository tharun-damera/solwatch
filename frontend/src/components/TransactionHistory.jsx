import { useState, useEffect, useRef } from "react";

import { Card, CardHeader, CardBody, CardFooter } from "./Card";
import { transactionHistory } from "../api/api";

export default function TransactionHistory({ address }) {
  const LIMIT = 20;

  const [txns, setTxns] = useState([]);
  const [skip, setSkip] = useState(0);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);

  const initialLoaded = useRef(false);

  async function fetchPage(skipParam) {
    if (loading) return;

    setLoading(true);

    try {
      const newTxns = await transactionHistory(address, skipParam, LIMIT);

      if (newTxns.length === 0) {
        setHasMore(false);
      } else {
        setTxns((prev) => [...prev, ...newTxns]);
        setSkip((prev) => prev + newTxns.length);

        if (newTxns.length < LIMIT) setHasMore(false);
      }
    } catch (err) {
      console.error("Error fetching transactions:", err);
    }

    setLoading(false);
  }

  async function reset_states() {
    setTxns([]);
    setSkip(0);
    setHasMore(true);

    // avoid double-fetch in React strict mode dev
    if (!initialLoaded.current) {
      initialLoaded.current = true;
      fetchPage(0);
    }
  }

  useEffect(() => {
    if (!address) return;

    reset_states();
  }, [address]);

  async function loadMore() {
    if (!hasMore || loading) return;
    await fetchPage(skip);
  }

  return (
    <Card>
      <CardHeader>Transaction History</CardHeader>
      <CardBody>
        <table>
          <thead>
            <tr>
              <th>Transaction Signature</th>
              <th>Block Time</th>
              <th>Slot</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {txns.map((txn) => (
              <tr key={txn._id}>
                <td>
                  <div className="truncated-text">
                    <a href="">{txn._id}</a>
                  </div>
                </td>
                <td>{txn.block_time}</td>
                <td>{txn.slot}</td>
                <td>{txn.confirmation_status.toUpperCase()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </CardBody>
      <CardFooter>
        {loading && (
          <p style={{ textAlign: "center", marginTop: "10px" }}>Loading...</p>
        )}

        <div
          style={{
            textAlign: "center",
            marginTop: "12px",
            marginBottom: "12px",
          }}
        >
          {!loading && hasMore && (
            <button className="sol-gradient-btn" onClick={loadMore}>
              Load More
            </button>
          )}

          {!hasMore && txns.length > 0 && (
            <p style={{ fontSize: "14px" }}>No more transactions</p>
          )}
        </div>
      </CardFooter>
    </Card>
  );
}
