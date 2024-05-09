module suifinity::ckSuiHelper{
    use sui::coin::{Self, Coin};
    use sui::event::emit;
    use sui::sui::SUI;
    use std::string::{String};

    public struct AdminCap has key, store { id: UID }

    public struct CkSuiMinter has key{
        id: UID,
        minter_address: address
    }

    public struct ReceivedSui has copy, drop, store {
        from: address,
        value: u64,
        principal_address: String,
        minter_address: address
    }

    fun init( ctx: &mut TxContext) {
        transfer::transfer(AdminCap{id: object::new(ctx)},tx_context::sender(ctx));
        transfer::transfer(CkSuiMinter { id: object::new(ctx), minter_address: @0x0}, tx_context::sender(ctx));
    }

    public fun getMinterAddress(minter: &mut CkSuiMinter): &mut address {
        &mut minter.minter_address
    }
    
    public fun setMinterAddress(_: &AdminCap, new_minter_address: address, minter: &mut CkSuiMinter){
        minter.minter_address = new_minter_address;
    }

    public fun deposit(value: u64, coin: &mut Coin<SUI>, principal_address: String, minter: &mut CkSuiMinter, ctx: &mut TxContext) {
        let balance = coin::balance_mut(coin);
        let new_coin = coin::take(balance, value, ctx);
        transfer::public_transfer(new_coin, minter.minter_address);
        let event = ReceivedSui {
            from: tx_context::sender(ctx),
            value: value,
            principal_address: principal_address,
            minter_address: minter.minter_address
        };
        emit(event);
    }

}