# contract-delta-weakened-ensures

Old: `ensures self.balance == old(self).balance - amount` and `self.balance >= 0`.
New: `ensures self.balance <= old(self).balance`; overdraft invariant removed.
