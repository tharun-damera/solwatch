import { useEffect, useState } from "react";
import { toTitleCase } from "../utils/case";

import { BASE_URL } from "../utils/env";

export default function TransactionDetails({
  address,
  signature,
  error,
  setError,
  onClose,
}) {
  const [txn, setTxn] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchTxn() {
      setError(null);
      setLoading(true);
      try {
        const res = await fetch(
          `${BASE_URL}/api/accounts/${address}/transactions/${signature}`,
          {
            method: "GET",
          }
        );
        if (!res.ok) throw new Error("Something went wrong");
        const data = await res.json();
        setTxn(data);
      } catch (e) {
        setError(e.message);
      } finally {
        setLoading(false);
      }
    }

    fetchTxn();
  }, [address, signature, setError]);

  if (error) return <></>;

  return (
    <div className="txn-overlay" onClick={onClose}>
      <div className="txn-modal" onClick={(e) => e.stopPropagation()}>
        <h2>Transaction Details</h2>
        <button className="txn-close" onClick={onClose}>
          ×
        </button>

        {loading && <div className="txn-loading">Loading…</div>}

        {!loading && txn && (
          <div className="txn-wrapper">
            {/* Signature */}
            <div className="txn-card">
              <h3>Signature</h3>
              <p className="break">{signature}</p>
            </div>

            {/* Status */}
            <div className="txn-card">
              <h3>Status</h3>
              <p>{txn.transaction.meta.err === null ? "Success" : "Failed"}</p>
            </div>

            {/* Basic Info */}
            <div className="txn-card">
              <h3>Basic Info</h3>
              <p>
                <strong>Slot:</strong> {txn.slot}
              </p>
              <p>
                <strong>Time:</strong>{" "}
                {new Date(txn.block_time * 1000).toLocaleString()}
              </p>
              <p>
                <strong>Fee:</strong> {txn.transaction.meta.fee} lamports
              </p>
            </div>

            {/* Account Keys */}
            <div className="txn-card">
              <h3>Accounts</h3>
              {txn.transaction.transaction.message.accountKeys.map((acc, i) => (
                <div key={i} className="txn-row">
                  <span className="break">{acc.pubkey}</span>
                  <span className="tag">
                    {acc.signer && "Signer"} {acc.writable && "Writable"}
                  </span>
                </div>
              ))}
            </div>

            {/* Instructions */}
            <div className="txn-card">
              <h3>Instructions</h3>
              {txn.transaction.transaction.message.instructions.map((ix, i) => (
                <div key={i} className="txn-instruction">
                  <strong>
                    {toTitleCase(ix.program)}: {toTitleCase(ix.parsed.type)}
                  </strong>
                  <p>
                    <strong>From:</strong>{" "}
                    <span className="break">{ix.parsed.info.source}</span>
                  </p>
                  <p>
                    <strong>To:</strong>{" "}
                    <span className="break">{ix.parsed.info.destination}</span>
                  </p>
                  <p>
                    <strong>Amount:</strong> {ix.parsed.info.lamports}
                  </p>
                </div>
              ))}
            </div>

            {/* Logs */}
            <div className="txn-card">
              <h3>Logs</h3>
              {txn.transaction.meta.logMessages.map((log, i) => (
                <pre key={i} className="txn-log">
                  {log}
                </pre>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
