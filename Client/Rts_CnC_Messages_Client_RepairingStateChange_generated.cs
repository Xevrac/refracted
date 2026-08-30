using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RepairingStateChange
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RepairingStateChange); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RepairingStateChange)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize RepairerEntityId
            s.Write(value.RepairerEntityId);
            //  Serialize TargetPlayerId
            s.Write(value.TargetPlayerId);
            //  Serialize TargetEntityId
            s.Write(value.TargetEntityId);
            //  Serialize RepairingState
            s.Write(value.RepairingState);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RepairingStateChange)) as Rts.CnC.Messages.Client.RepairingStateChange;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize RepairerEntityId
            s.Read(out value.RepairerEntityId);
            //  Deserialize TargetPlayerId
            s.Read(out value.TargetPlayerId);
            //  Deserialize TargetEntityId
            s.Read(out value.TargetEntityId);
            //  Deserialize RepairingState
            s.Read(out value.RepairingState);

            return value;
        }
        
    }
}
