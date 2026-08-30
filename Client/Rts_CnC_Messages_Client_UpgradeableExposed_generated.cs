using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_UpgradeableExposed
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.UpgradeableExposed); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.UpgradeableExposed)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize array UnlockedUpgrades
            Rts.Serialization.Reference.Write(s, value.UnlockedUpgrades, () =>
            {
                s.WriteVarInt32(value.UnlockedUpgrades.Length);
                for(int i = 0 ; i < value.UnlockedUpgrades.Length ; ++i)
                {
                    s.Write(value.UnlockedUpgrades[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.UpgradeableExposed)) as Rts.CnC.Messages.Client.UpgradeableExposed;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize array UnlockedUpgrades
            Rts.Serialization.Reference.Read(s, out value.UnlockedUpgrades, () =>
            {
                int length = s.ReadVarInt32();
                System.UInt32[] tmp = new System.UInt32[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
