script {
    use 0x1::BasicCoin;

    fun main(me: signer) {
        BasicCoin::mint(me, 200);
    }
}
