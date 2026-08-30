using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestFWAMove
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestFWAMove); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestFWAMove)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize AirfieldId
            s.Write(value.AirfieldId);
            //  Serialize TargetLocation
            s.Write(value.TargetLocation);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestFWAMove)) as Rts.CnC.Messages.Client.RequestFWAMove;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize AirfieldId
            s.Read(out value.AirfieldId);
            //  Deserialize TargetLocation
            s.Read(out value.TargetLocation);

            return value;
        }
        
    }
}
