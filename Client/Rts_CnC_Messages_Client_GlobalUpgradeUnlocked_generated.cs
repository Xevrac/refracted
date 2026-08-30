using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_GlobalUpgradeUnlocked
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.GlobalUpgradeUnlocked); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.GlobalUpgradeUnlocked)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize Upgrade
            s.Write(value.Upgrade);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.GlobalUpgradeUnlocked)) as Rts.CnC.Messages.Client.GlobalUpgradeUnlocked;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize Upgrade
            s.Read(out value.Upgrade);

            return value;
        }
        
    }
}
