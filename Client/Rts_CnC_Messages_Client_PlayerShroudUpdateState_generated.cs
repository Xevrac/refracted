using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlayerShroudUpdateState
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlayerShroudUpdateState); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlayerShroudUpdateState)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize IsSendingUpdate
            s.Write(value.IsSendingUpdate);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlayerShroudUpdateState)) as Rts.CnC.Messages.Client.PlayerShroudUpdateState;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize IsSendingUpdate
            s.Read(out value.IsSendingUpdate);

            return value;
        }
        
    }
}
