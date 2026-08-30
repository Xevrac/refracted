using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_UpgradeAlert
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.UpgradeAlert); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.UpgradeAlert)obj;
            //  Serialize UpgradeId
            s.Write(value.UpgradeId);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.UpgradeAlert)) as Rts.CnC.Messages.Client.UpgradeAlert;
            //  Deserialize UpgradeId
            s.Read(out value.UpgradeId);

            return value;
        }
        
    }
}
